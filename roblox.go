package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"strconv"
	"strings"

	"github.com/zalando/go-keyring"
)

type User struct {
	Id          uint   `json:"id"`
	Name        string `json:"name"`
	DisplayName string `json:"displayName"`
}

type UserPresence struct {
	UserPresenceType int     `json:"userPresenceType"`
	LastLocation     string  `json:"lastLocation"`
	PlaceId          *int    `json:"placeId"`
	RootPlaceId      *int    `json:"rootPlaceId"`
	GameId           *string `json:"gameId"`
	UniverseId       *int    `json:"universeId"`
	UserId           int     `json:"userId"`
	LastOnline       string  `json:"lastOnline"`
}

type presenceRoot struct {
	UserPresences []UserPresence `json:"userPresences"`
}

type ImageData struct {
	TargetId int    `json:"targetId"`
	State    string `json:"state"`
	ImageUrl string `json:"imageUrl"`
}

type imageDataRoot struct {
	ImageData []ImageData `json:"data"`
}

var (
	rbxUser User

	ErrNotAuthenticated       = errors.New("user is not authenticated to roblox")
	ErrGameIdNotFound         = errors.New("unable to get game id")
	ErrTicketRedemption       = errors.New("could not redeem authentication ticket")
	ErrSecurityCookieNotFound = errors.New("response did not contain .ROBLOSECURITY cookie")
)

func GetUserPresence() (*UserPresence, error) {
	presenceData := &presenceRoot{}

	userid := strconv.FormatUint(uint64(rbxUser.Id), 10)

	body := strings.NewReader(`{"userIds":[` + userid + `]}`)
	req, err := http.NewRequest("POST", "https://presence.roblox.com/v1/presence/users", body)
	if err != nil {
		return &UserPresence{}, err
	}

	token, err := keyring.Get("RblxRichPresence", "token")
	if err != nil {
		return nil, fmt.Errorf("failed to get security token from keyring: %v", err)
	}

	req.AddCookie(&http.Cookie{
		Name:  ".ROBLOSECURITY",
		Value: token,
	})

	req.Header.Add("Content-Type", "application/json")

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return &UserPresence{}, err
	}

	if resp.StatusCode != http.StatusOK {
		return &UserPresence{}, ErrGameIdNotFound
	}

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return &UserPresence{}, err
	}
	defer resp.Body.Close()

	if err = json.Unmarshal(data, presenceData); err != nil {
		return &UserPresence{}, err
	}

	return &presenceData.UserPresences[0], nil
}

func GetAuthenticatedUser() (*User, error) {
	if rbxUser.Id != uint(0) {
		return &rbxUser, nil
	}

	req, err := http.NewRequest("GET", "https://users.roblox.com/v1/users/authenticated", nil)
	if err != nil {
		return nil, err
	}

	token, err := keyring.Get("RblxRichPresence", "token")
	if err != nil {
		return nil, err
	}

	req.AddCookie(&http.Cookie{
		Name:  ".ROBLOSECURITY",
		Value: token,
	})

	client := &http.Client{}

	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, ErrNotAuthenticated
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	if err = json.Unmarshal(body, &rbxUser); err != nil {
		return nil, err
	}

	return &rbxUser, nil
}

func RefreshSecurityCookie(joinData *DirectJoinData) error {
	// Re-authenticate the user
	body := strings.NewReader(`{"authenticationTicket": "` + joinData.gameInfo + `"}`)
	req, err := http.NewRequest("POST", "https://auth.roblox.com/v1/authentication-ticket/redeem", body)
	if err != nil {
		return err
	}

	req.Header.Add("Content-Type", "application/json")
	req.Header.Add("User-Agent", "RobloxStudio/WinInet")
	req.Header.Add("Accept", "application/json")
	req.Header.Add("RBXAuthenticationNegotiation", "https://www.roblox.com/")

	client := &http.Client{}

	resp, err := client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return ErrTicketRedemption
	}

	for _, cookie := range resp.Cookies() {
		if cookie.Name != ".ROBLOSECURITY" {
			continue
		}

		if err := keyring.Set("RblxRichPresence", "token", cookie.Value); err != nil {
			return err
		}

		ticket, err := GetAuthenticationTicket()
		if err != nil {
			return err
		}

		joinData.gameInfo = ticket
		return nil
	}

	return ErrSecurityCookieNotFound
}

func GetAuthenticationTicket() (string, error) {
	req, err := http.NewRequest("POST", "https://auth.roblox.com/v1/authentication-ticket", nil)
	if err != nil {
		return "", err
	}

	req.Header.Add("Content-Type", "application/json")
	req.Header.Add("Accept", "application/json")
	req.Header.Add("Referer", "https://www.roblox.com/")

	token, err := keyring.Get("RblxRichPresence", "token")
	if err != nil {
		return "", err
	}

	req.AddCookie(&http.Cookie{
		Name:  ".ROBLOSECURITY",
		Value: token,
	})

	client := &http.Client{}

	var resp *http.Response
	for {
		resp, err = client.Do(req)
		if err != nil {
			return "", err
		}
		defer resp.Body.Close()

		// Get new x-csrf-token if provided one was invalid
		if resp.StatusCode == http.StatusForbidden {
			req.Header.Add("x-csrf-token", resp.Header.Get("x-csrf-token"))
		} else {
			break
		}
	}

	return resp.Header.Get("rbx-authentication-ticket"), nil
}

func GetExperienceIcon(universeId int) (string, error) {
	endpoint := "https://thumbnails.roblox.com/v1/games/icons?universeIds=%d&returnPolicy=PlaceHolder&size=512x512&format=Png&isCircular=false"

	req, err := http.NewRequest("GET", fmt.Sprintf(endpoint, universeId), nil)
	if err != nil {
		return "", err
	}

	req.Header.Add("Content-Type", "application/json")
	req.Header.Add("Accept", "application/json")

	client := &http.Client{}

	var resp *http.Response

	resp, err = client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("could not get experience icon: %v", err)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}

	iconData := &imageDataRoot{}
	if err = json.Unmarshal(body, &iconData); err != nil {
		return "", err
	}

	return *&iconData.ImageData[0].ImageUrl, nil
}
