package main

import (
	"strconv"
	"time"

	"github.com/hugolgst/rich-go/client"
)

func StartDiscordRpc() error {
	err := client.Login("725360592570941490")
	if err != nil {
		return err
	}

	user, err := GetAuthenticatedUser()
	if err != nil {
		// User is not authenticated (most likely the security cookie has expired)
		// TODO: notify user about invalid cookie
		return err
	}

	config, err := GetConfig()
	if err != nil {
		return err
	}

	var largeText string
	if config.RichPresence.DisplayUsername {
		largeText = "Playing Roblox as " + user.Name
	} else {
		largeText = "Playing Roblox"
	}

	// Update loop
	var lastPresence *UserPresence

	for {
		presence, err := GetUserPresence()
		if err != nil || presence.LastLocation == "Website" {
			time.Sleep(3 * time.Second)
			continue
		}

		// User has joined a different game/server through in-game teleportation
		if lastPresence == nil || *presence.GameId != *lastPresence.GameId {
			startTime := time.Now()
			placeId := strconv.Itoa(*presence.RootPlaceId)

			client.SetActivity(client.Activity{
				State:   "In an experience",
				Details: "Playing " + presence.LastLocation,

				LargeImage: "logo",
				LargeText:  largeText,

				SmallImage: "play_status",
				SmallText:  presence.LastLocation,

				Buttons: []*client.Button{
					{
						Label: "View on website",
						Url:   "https://www.roblox.com/games/" + placeId,
					},
					{
						Label: "Launch Roblox",

						// Discord seems to be messing up the protocol link, going to use the web link instead
						// Expected: roblox://placeId=123456
						// Observed: roblox://placeid/=123456
						Url: "https://www.roblox.com/games/start?placeId=" + placeId,
					},
				},

				Timestamps: &client.Timestamps{
					Start: &startTime,
				},
			})

			lastPresence = presence
		}

		// Discord Rich Presence has a ratelimit of 1 update per 15 seconds
		time.Sleep(15 * time.Second)
	}
}
