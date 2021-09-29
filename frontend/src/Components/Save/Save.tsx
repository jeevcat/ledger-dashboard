import React, { useEffect, useState } from "react";
import { Button, Form, Grid, Header, Label, Segment } from "semantic-ui-react";
import { getDirtyJournalFiles, saveJournal } from "../../Utils/BackendRequester";
import { DirectoryListing } from "./DirectoryListing";

const nameKey = "save_name";
const emailKey = "save_email";

interface Props {}

export const Save: React.FC<Props> = () => {
  const [message, setMessage] = useState("");
  const [name, setName] = useState(localStorage.getItem(nameKey) ?? "");
  const [email, setEmail] = useState(localStorage.getItem(emailKey) ?? "");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [dirtyFiles, setDirtyFiles] = useState<string[]>([]);

  const updateDirtyFiles = () => {
    setLoading(true);
    getDirtyJournalFiles()
      .then(setDirtyFiles)
      .then(() => setLoading(false));
  };

  useEffect(() => {
    updateDirtyFiles();
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (message) {
      setLoading(true);
      localStorage.setItem(nameKey, name);
      localStorage.setItem(emailKey, email);
      saveJournal({ commitMsg: message, name, email }).then(updateDirtyFiles);
    } else {
      setError("Commit message must not be empty.");
    }
  };

  return (
    <Grid textAlign="center" verticalAlign="middle" style={{ height: "100vh", margin: 0 }}>
      <Grid.Column style={{ maxWidth: 800 }} textAlign="left">
        <Header as="h1">Save changes</Header>
        <Form size="large" onSubmit={handleSubmit}>
          <Segment stacked>
            <Form.Field error={!!error}>
              <input
                placeholder="Commit message"
                type="text"
                onChange={(e) => {
                  setMessage(e.target.value);
                  setError("");
                }}
              />
              {error && (
                <Label color="red" pointing>
                  {error}
                </Label>
              )}
            </Form.Field>
            <Form.Field>
              <input
                placeholder="Name"
                type="text"
                onChange={(e) => {
                  setName(e.target.value);
                }}
              />
            </Form.Field>
            <Form.Field>
              <input
                placeholder="E-mail"
                type="text"
                onChange={(e) => {
                  setEmail(e.target.value);
                }}
              />
            </Form.Field>
            <DirectoryListing paths={dirtyFiles} />

            <Button color="blue" fluid size="large" loading={loading} disabled={dirtyFiles.length === 0}>
              Commit and push
            </Button>
          </Segment>
        </Form>
      </Grid.Column>
    </Grid>
  );
};
