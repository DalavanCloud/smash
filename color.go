package main

type Color struct {
	R, G, B byte
}

var black = Color{0, 0, 0}

var white = Color{0xff, 0xff, 0xff}

var ansiColors = [...]Color{
	{0x2e, 0x34, 0x36},
	{0xcc, 0x00, 0x00},
	{0x4e, 0x9a, 0x06},
	{0xc4, 0xa0, 0x00},
	{0x34, 0x65, 0xa4},
	{0x75, 0x50, 0x7b},
	{0x06, 0x98, 0x9a},
	{0xd3, 0xd7, 0xcf},
}

var ansiBrightColors = [...]Color{
	{0x55, 0x57, 0x53},
	{0xef, 0x29, 0x29},
	{0x8a, 0xe2, 0x34},
	{0xfc, 0xe9, 0x4f},
	{0x72, 0x9f, 0xcf},
	{0xad, 0x7f, 0xa8},
	{0x34, 0xe2, 0xe2},
	{0xee, 0xee, 0xec},
}