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

	// Should probably wait for a few seconds as the presence returns Website as we are fetching it too quickly
	lastPresence, err := GetUserPresence()
	if err != nil {
		return err
	}

	config, err := GetConfig()
	if err != nil {
		return err
	}

	var largeText string
	if config.RichPresence.DisplayGame {
		largeText = "Playing Roblox as " + user.Name
	} else {
		largeText = "Playing Roblox"
	}

	now := time.Now()
	err = client.SetActivity(client.Activity{
		State:   "In an experience",
		Details: "Playing " + lastPresence.LastLocation,

		LargeImage: "logo",
		LargeText:  largeText,

		SmallImage: "play_status",
		SmallText:  lastPresence.LastLocation,

		Buttons: []*client.Button{
			{
				Label: "View on Roblox",
				Url:   "https://www.roblox.com/games/" + strconv.Itoa(*lastPresence.RootPlaceId),
			},
		},

		Timestamps: &client.Timestamps{
			Start: &now,
		},
	})

	if err != nil {
		return err
	}

	// Update loop
	// Is there a better way to implement this?
	for {
		// Discord Rich Presence has a ratelimit of 1 update per 15 seconds
		time.Sleep(15 * time.Second)

		presence, err := GetUserPresence()
		if err != nil {
			continue
		}

		if presence.LastLocation == "Website" {
			client.Logout()
			return nil
		}

		if lastPresence == nil || lastPresence.LastLocation == "Website" {
			lastPresence = presence
		}

		// User has joined a different game/server through in-game teleportation
		if presence.GameId != lastPresence.GameId {
			startTime := time.Now()

			client.SetActivity(client.Activity{
				State:   "In an experience",
				Details: "Playing " + presence.LastLocation,

				LargeImage: "logo",
				LargeText:  largeText,

				SmallImage: "play_status",
				SmallText:  presence.LastLocation,

				Buttons: []*client.Button{
					{
						Label: "View on Roblox",
						Url:   "https://www.roblox.com/games/" + strconv.Itoa(*lastPresence.RootPlaceId),
					},
				},

				Timestamps: &client.Timestamps{
					Start: &startTime,
				},
			})

			lastPresence = presence
		}
	}
}
