// generated by stringer -type=Sym; DO NOT EDIT

package keys

import "fmt"

const (
	_Sym_name_0 = "NoSym"
	_Sym_name_1 = "BackspaceTab"
	_Sym_name_2 = "Enter"
	_Sym_name_3 = "LeftRightUpDown"
)

var (
	_Sym_index_0 = [...]uint8{0, 5}
	_Sym_index_1 = [...]uint8{0, 9, 12}
	_Sym_index_2 = [...]uint8{0, 5}
	_Sym_index_3 = [...]uint8{0, 4, 9, 11, 15}
)

func (i Sym) String() string {
	switch {
	case i == 0:
		return _Sym_name_0
	case 8 <= i && i <= 9:
		i -= 8
		return _Sym_name_1[_Sym_index_1[i]:_Sym_index_1[i+1]]
	case i == 13:
		return _Sym_name_2
	case 132 <= i && i <= 135:
		i -= 132
		return _Sym_name_3[_Sym_index_3[i]:_Sym_index_3[i+1]]
	default:
		return fmt.Sprintf("Sym(%d)", i)
	}
}
