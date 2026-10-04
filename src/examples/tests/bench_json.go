package main

import (
	"encoding/json"
	"fmt"
)

type Item struct {
	ID   int    `json:"id"`
	Name string `json:"name"`
}

func main() {
	items := make([]Item, 1000)
	for i := range items {
		items[i] = Item{ID: i, Name: "item"}
	}
	data, _ := json.Marshal(items)
	var back []Item
	_ = json.Unmarshal(data, &back)
	again, _ := json.Marshal(back)
	fmt.Printf("count=%d len=%d\n", len(back), len(again))
}
