package controller

import (
	"github.com/thavlik/homesec/controller/pkg/controller/temperaturesensor"
)

func init() {
	// AddToManagerFuncs is a list of functions to create controllers and add them to a manager.
	AddToManagerFuncs = append(AddToManagerFuncs, temperaturesensor.Add)
}
