package credentials

import (
	"time"

	"github.com/golang-jwt/jwt/v5"
	log "github.com/sirupsen/logrus"
)

type AuthCredential interface {
	// CredentialType interface
	// Get the credential type
	GetToken() (*Token, error)
}

type Token struct {
	// Token fields
	AccessToken  string
	ExpiresOn    time.Time // Stored in local time
	RefreshToken string
}

// Check if token is expired
func (t *Token) IsExpired() bool {
	if t.ExpiresOn.Before(time.Now()) {

		// Only log if the token has an expiration time set
		if !t.ExpiresOn.IsZero() {
			log.Info("Token is expired. Obtaining new one.")
		}
		return true
	} else {
		return false
	}
}

func (t *Token) SetExpiresOnFromAccessToken() error {
	// Get the expiration time from the access token
	// Parse access token to get expiration time
	parsedToken, _, err := jwt.NewParser().ParseUnverified(t.AccessToken, jwt.MapClaims{})
	if err != nil {
		return err
	}
	t.ExpiresOn = time.Unix(int64(parsedToken.Claims.(jwt.MapClaims)["exp"].(float64)), 0)
	return nil
}

func (t *Token) GetClaim(claim string) (string, error) {
	// Parse access token to get the oid claim
	parsedToken, _, err := jwt.NewParser().ParseUnverified(t.AccessToken, jwt.MapClaims{})
	if err != nil {
		return "", err
	}
	return parsedToken.Claims.(jwt.MapClaims)[claim].(string), nil
}
