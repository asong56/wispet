package ecdict

import "errors"

// Status reports the entry count and edition ("compact" or "full") of
// whatever ECDICT-format database is currently loaded. Medict no longer
// ships a bundled database or a full-edition downloader; users who want the
// complete ECDICT dataset can build/obtain an ecdict.db themselves (see
// https://github.com/skywind3000/ECDICT) and drop it into the dictionary
// directory like any other dictionary.
type Status struct {
	EntryCount int    `json:"entryCount"`
	Edition    string `json:"edition"`
}

func (e *ECDict) Status() (Status, error) {
	e.mu.RLock()
	defer e.mu.RUnlock()
	if e.db == nil {
		return Status{}, errors.New("ecdict: database closed")
	}
	var status Status
	if err := e.db.QueryRow(`SELECT COUNT(*) FROM ecdict`).Scan(&status.EntryCount); err != nil {
		return Status{}, err
	}
	status.Edition = "compact"
	var edition string
	if err := e.db.QueryRow(`SELECT value FROM medict_meta WHERE key = 'edition'`).Scan(&edition); err == nil && edition != "" {
		status.Edition = edition
	}
	return status, nil
}
