package credentials

import (
	"crypto/sha1"
	"crypto/x509"
	"encoding/base64"
	"encoding/pem"
	"fmt"
	"net/http"
	"net/url"
	"os"
	"strings"
	"sync"
	"time"

	"github.com/Jeffail/gabs/v2"
	"github.com/golang-jwt/jwt/v5"
	"github.com/google/uuid"
	log "github.com/sirupsen/logrus"
)

type ClientCredentialCert struct {
	mu              sync.RWMutex
	TokenEndpoint   string
	Resource        string
	TenantID        string
	ClientId        string
	CertificatePath string
	Token           Token
}

func (c *ClientCredentialCert) generateClientAssertion() (*string, error) {
	var err error

	data, err := os.ReadFile(c.CertificatePath)
	if err != nil {
		return nil, err
	}

	// Parse PEM blocks
	var certificateBlock *x509.Certificate
	var privateKey interface{}

	for {
		var block *pem.Block
		block, data = pem.Decode(data)
		if block == nil {
			break
		}

		switch block.Type {
		case "CERTIFICATE":
			certificateBlock, err = x509.ParseCertificate(block.Bytes)
			if err != nil {
				return nil, err
			}
		case "RSA PRIVATE KEY":
			privateKey, err = x509.ParsePKCS1PrivateKey(block.Bytes)
			if err != nil {
				return nil, err
			}
		case "PRIVATE KEY":
			privateKey, err = x509.ParsePKCS8PrivateKey(block.Bytes)
			if err != nil {
				return nil, err
			}
		}
	}

	// Get the base64 string of the certificate SHA1 thumbprint
	thumbprint := sha1.New()
	thumbprint.Write(certificateBlock.Raw)
	x5t := base64.StdEncoding.EncodeToString(thumbprint.Sum(nil))

	token := jwt.New(jwt.SigningMethodRS256)
	token.Header = map[string]interface{}{
		"alg": "RS256",
		"typ": "JWT",
		"x5t": x5t,
	}
	token.Claims = jwt.MapClaims{
		"iss": c.ClientId,
		"sub": c.ClientId,
		"aud": fmt.Sprintf("%s/%s/oauth2/token", c.TokenEndpoint, c.TenantID),
		"exp": time.Now().Add(10 * time.Minute).Unix(),
		"nbf": time.Now().Unix(),
		"jti": uuid.New().String(),
	}

	clientAssertion, err := token.SignedString(privateKey)
	return &clientAssertion, err
}

// Gets the token for the Azure CLI credential
func (c *ClientCredentialCert) GetToken() (*Token, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	if c.Token.IsExpired() {

		tokenUrl := fmt.Sprintf("%s/%s/oauth2/v2.0/token", c.TokenEndpoint, c.TenantID)
		clientAssertion, err := c.generateClientAssertion()
		if err != nil {
			log.Error(err)
			return nil, err
		}

		// Authenticate to login.microsoftonline.com with client secret via HTTP POST
		data := url.Values{}
		data.Set("grant_type", "client_credentials")
		data.Set("client_id", c.ClientId)
		data.Set("client_assertion", *clientAssertion)
		data.Set("client_assertion_type", "urn:ietf:params:oauth:client-assertion-type:jwt-bearer")
		data.Set("scope", c.Resource+"/.default")

		request, err := http.NewRequest(http.MethodPost, tokenUrl, strings.NewReader(data.Encode()))
		if err != nil {
			return nil, err
		}

		request.Header.Set("Content-Type", "application/x-www-form-urlencoded")
		request.Header.Set("Authorization", "Bearer "+*clientAssertion)

		resp, err := http.DefaultClient.Do(request)
		if err != nil {
			log.Error(err)
			return nil, err
		}
		defer resp.Body.Close()

		jsonParsed, err := gabs.ParseJSONBuffer(resp.Body)
		if err != nil {
			return nil, err
		}

		if jsonParsed.Exists("error_description") {
			return nil, fmt.Errorf("Error: %s", jsonParsed.Path("error_description").Data().(string))
		}

		c.Token.AccessToken = jsonParsed.Path("access_token").Data().(string)
		if err := c.Token.SetExpiresOnFromAccessToken(); err != nil {
			return nil, err
		}
	}
	return &c.Token, nil
}
