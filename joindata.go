package main

import (
	"net/url"
	"strings"
)

type DirectJoinData struct {
	launchMode       string
	gameInfo         string
	placeLauncherUrl string
	robloxLocale     string
	gameLocale       string
	browserTrackerId string
}

type DeeplinkJoinData struct {
	placeId string
	gameId  string
}

func unmarshallJoinUrl(joinUrl string) (DirectJoinData, error) {
	joinData := DirectJoinData{}

	for _, option := range strings.Split(joinUrl, "+") {
		pair := strings.Split(option, ":")
		if len(pair) < 2 {
			continue
		}

		name, value := pair[0], pair[1]

		switch name {
		case "launchmode":
			joinData.launchMode = value

		case "gameinfo":
			joinData.gameInfo = value

		case "placelauncherurl":
			escapedValue, err := url.QueryUnescape(value)
			if err != nil {
				return joinData, err
			}

			joinData.placeLauncherUrl = escapedValue
		case "robloxLocale":
			joinData.robloxLocale = value

		case "gameLocale":
			joinData.gameLocale = value

		case "browsertrackerid":
			joinData.browserTrackerId = value

		default:
			continue
		}
	}

	return joinData, nil
}
