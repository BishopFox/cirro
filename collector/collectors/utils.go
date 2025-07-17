package collectors

func ContainsAny[T comparable](slice1, slice2 []T) bool {
	// Create a map to store the elements of slice1 for quick lookup
	elementMap := make(map[T]struct{})
	for _, elem := range slice1 {
		elementMap[elem] = struct{}{}
	}

	// Check if any element of slice2 is in slice1
	for _, elem := range slice2 {
		if _, found := elementMap[elem]; found {
			return true
		}
	}
	return false
}
