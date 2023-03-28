package main

import (
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"strconv"
	"strings"
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

type PresenceRoot struct {
	UserPresences []UserPresence `json:"userPresences"`
}

var rbxUser User

func GetUserPresence() (*UserPresence, error) {
	presenceData := &PresenceRoot{}

	userid := strconv.FormatUint(uint64(rbxUser.Id), 10)

	body := strings.NewReader(`{"userIds":[` + userid + `]}`)
	req, err := http.NewRequest("POST", "https://presence.roblox.com/v1/presence/users", body)
	if err != nil {
		return &UserPresence{}, err
	}

	config, err := GetConfig()
	if err != nil {
		return &UserPresence{}, err
	}

	req.AddCookie(&http.Cookie{
		Name:  ".ROBLOSECURITY",
		Value: config.Roblox.RbxSecurity,
	})

	req.Header.Add("Content-Type", "application/json")

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return &UserPresence{}, err
	}

	if resp.StatusCode != http.StatusOK {
		return &UserPresence{}, errors.New("unable to get game id")
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

	config, err := GetConfig()
	if err != nil {
		return nil, err
	}

	req.AddCookie(&http.Cookie{
		Name:  ".ROBLOSECURITY",
		Value: config.Roblox.RbxSecurity,
	})

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}

	if resp.StatusCode != http.StatusOK {
		return nil, errors.New("user is not authenticated to roblox")
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
