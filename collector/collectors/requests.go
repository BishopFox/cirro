package collectors

import (
	"fmt"
	"io"
	"net/http"
	"strings"

	"github.com/Jeffail/gabs/v2"
	"github.com/bishopfox/cirro/collector/credentials"
	log "github.com/sirupsen/logrus"
)

// pagedRequest makes a request to the Graph API and returns the response
func pagedRequest(cred credentials.AuthCredential, resourceUrl string, uri string) (*gabs.Container, http.Header, error) {
	var requestUrl string

	if strings.HasPrefix(uri, resourceUrl) {
		requestUrl = uri
	} else {
		requestUrl = fmt.Sprintf("%s/%s", resourceUrl, uri)
	}
	log.Debug("Paged request to " + requestUrl)

	token, err := cred.GetToken()
	if err != nil {
		return nil, nil, err
	}

	request, err := http.NewRequest(http.MethodGet, requestUrl, nil)
	if err != nil {
		return nil, nil, err
	}
	request.Header.Set("Authorization", "Bearer "+token.AccessToken)

	resp, err := http.DefaultClient.Do(request)
	if err != nil {
		return nil, nil, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if resp.StatusCode != 200 {
		return nil, resp.Header, fmt.Errorf("HTTP status code: %d - %s", resp.StatusCode, string(body))
	}
	if err != nil {
		return nil, nil, err
	}

	parsed, err := gabs.ParseJSON(body)
	if err != nil {
		return nil, resp.Header, err
	}

	if parsed.Exists("@odata.error") {
		return nil, resp.Header, fmt.Errorf(parsed.Path("@odata.error.message").Data().(string))
	}

	return parsed, resp.Header, nil
}
