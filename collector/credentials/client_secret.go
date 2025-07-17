package credentials

import (
	"fmt"
	"net/http"
	"net/url"
	"sync"

	"github.com/Jeffail/gabs/v2"
	log "github.com/sirupsen/logrus"
)

type ClientCredentialSecret struct {
	mu            sync.RWMutex
	TokenEndpoint string
	Resource      string
	TenantID      string
	ClientId      string
	ClientSecret  string
	Token         Token
}

// Gets the token for the Azure CLI credential
func (c *ClientCredentialSecret) GetToken() (*Token, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	if c.Token.IsExpired() {

		tokenUrl := fmt.Sprintf("%s/%s/oauth2/v2.0/token", c.TokenEndpoint, c.TenantID)

		// Authenticate to login.microsoftonline.com with client secret via HTTP POST
		data := url.Values{}
		data.Set("grant_type", "client_credentials")
		data.Set("client_id", c.ClientId)
		data.Set("client_secret", c.ClientSecret)
		data.Set("scope", c.Resource+"./default")

		resp, err := http.PostForm(tokenUrl, data)
		if err != nil {
			log.Error(err)
			return nil, err
		}
		defer resp.Body.Close()

		jsonParsed, err := gabs.ParseJSONBuffer(resp.Body)
		if err != nil {
			log.Error(err)
		}

		c.Token.AccessToken = jsonParsed.Path("access_token").Data().(string)

		if err := c.Token.SetExpiresOnFromAccessToken(); err != nil {
			log.Error(err)
			return nil, err
		}
	}
	return &c.Token, nil
}
