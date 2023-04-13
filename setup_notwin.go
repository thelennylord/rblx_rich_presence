//go:build !windows

package main

import "errors"

// See https://github.com/thelennylord/rblx_rich_presence/issues/9
// and https://github.com/thelennylord/rblx_rich_presence/issues/10
func setup() error {
	return errors.New("Roblox Rich Presence currently only supports Windows")
}
