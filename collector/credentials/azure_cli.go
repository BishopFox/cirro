package credentials

import (
	"os"
	"os/exec"
	"strings"
	"sync"

	"github.com/Jeffail/gabs/v2"
	log "github.com/sirupsen/logrus"
)

type AzureCliCredential struct {
	mu sync.RWMutex
	// Azure CLI credential
	Resource string
	TenantID string
	Token    Token
}

// Gets the token for the Azure CLI credential
func (c *AzureCliCredential) GetToken() (*Token, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	if c.Token.IsExpired() {

		commandLine := "az account get-access-token -o json --resource " + c.Resource
		if c.TenantID != "" {
			commandLine += " --tenant " + c.TenantID
		}

		// Execute the command silently and store the output in a result variable
		args := strings.Fields(commandLine)
		process := exec.Command(args[0], args[1:]...)
		process.Stderr = os.Stderr

		out, err := process.Output()
		if err != nil {
			log.Error(err)
			return nil, err
		}

		jsonParsed, err := gabs.ParseJSON(out)
		if err != nil {
			log.Error(err)
			return nil, err
		}

		c.Token.AccessToken = jsonParsed.Path("accessToken").Data().(string)
		if err := c.Token.SetExpiresOnFromAccessToken(); err != nil {
			log.Error(err)
			return nil, err
		}
	}
	return &c.Token, nil
}
