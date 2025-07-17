package credentials

import (
	"errors"

	log "github.com/sirupsen/logrus"
)

type AccessTokenCredential struct {
	Token Token
}

// Gets the token for the Azure CLI credential
func (c *AccessTokenCredential) GetToken() (*Token, error) {

	if c.Token.IsExpired() {
		// Exit the program if the token is expired
		log.Error("Token is expired")
		return nil, errors.New("Token is expired")
	}
	return &c.Token, nil
}
