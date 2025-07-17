package collectors

import (
	"database/sql"
	"path/filepath"

	log "github.com/sirupsen/logrus"
	_ "modernc.org/sqlite"
)

type CirroDB struct {
	db *sql.DB
	tx *sql.Tx
}

func NewCirroDB(dbPath string) (*CirroDB, error) {
	if filepath.Ext(dbPath) != ".db" {
		dbPath = dbPath + ".db"
	}

	db, err := sql.Open("sqlite", dbPath)
	if err != nil {
		return nil, err
	}
	return &CirroDB{db: db}, nil
}

func (c *CirroDB) Init() error {
	var err error

	_, err = c.db.Exec(`PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA automatic_index = true;`)
	if err != nil {
		return err
	}
	c.tx, err = c.db.Begin()
	if err != nil {
		return err
	}
	return nil
}

func (c *CirroDB) Commit() error {
	err := c.tx.Commit()
	if err != nil {
		return err
	}
	c.tx, err = c.db.Begin()
	if err != nil {
		return err
	}
	return nil
}

func (c *CirroDB) Close() error {
	if c.Commit() != nil {
		log.Error("Failed to commit transaction. Attempting to rollback.")
		return c.tx.Rollback()
	}
	if c.tx != nil {
		c.tx.Exec("PRAGMA wal_checkpoint(FULL);")
	}
	return c.db.Close()
}

func (c *CirroDB) Exec(statement string, args ...any) error {
	_, err := c.tx.Exec(statement, args...)
	if err != nil {
		return err
	}
	return nil
}
func (c *CirroDB) PrepareTx(statement string) (*sql.Stmt, error) {
	return c.tx.Prepare(statement)
}

func (c *CirroDB) Query(statement string, args ...any) (*sql.Rows, error) {
	return c.db.Query(statement, args...)
}

func (c *CirroDB) QueryTx(statement string, args ...any) (*sql.Rows, error) {
	return c.tx.Query(statement, args...)
}

func (c *CirroDB) QueryRow(statement string, args ...any) *sql.Row {
	return c.db.QueryRow(statement, args...)
}

func (c *CirroDB) QueryRowTx(statement string, args ...any) (*sql.Row, error) {
	return c.tx.QueryRow(statement, args...), nil
}
