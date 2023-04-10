package main

import (
	"log"
	"strconv"
	"time"

	drpc "github.com/thelennylord/go-discordrpc"
)

var ClientId = "725360592570941490"

func setPresence(client *drpc.Client) {
	user, err := GetAuthenticatedUser()
	if err != nil {
		// User is not authenticated (most likely the security cookie has expired)
		// TODO: notify user about invalid cookie
		log.Fatal(err)
	}

	config, err := GetConfig()
	if err != nil {
		log.Fatal(err)
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
			placeId := strconv.Itoa(*presence.RootPlaceId)

			imageUrl, err := GetExperienceIcon(*presence.UniverseId)
			if err != nil {
				return
			}

			err = client.SetActivity(drpc.Activity{
				State:   "In an experience",
				Details: "Playing " + presence.LastLocation,

				Assets: &drpc.Assets{
					LargeImage: imageUrl,
					LargeText:  largeText,

					SmallImage: "play_status",
					SmallText:  presence.LastLocation,
				},

				Buttons: []*drpc.Button{
					{
						Label: "View on website",
						Url:   "https://www.roblox.com/games/" + placeId,
					},
					{
						Label: "Launch Roblox",
						Url:   "roblox://experiences/start?placeId=" + placeId,
					},
				},

				Timestamps: &drpc.Timestamps{
					Start: &drpc.Epoch{Time: time.Now()},
				},
			})
			if err != nil {
				log.Fatal(err)
			}

			lastPresence = presence
		}

		// Discord Rich Presence has a ratelimit of 5 updates per 20 seoconds
		// So checking every 5 seconds seems resonable
		time.Sleep(5 * time.Second)
	}
}
