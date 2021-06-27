import React, { useEffect, useState } from "react";
import { Button, Form, Grid, Header, Label, Segment } from "semantic-ui-react";
import { getDirtyJournalFiles, saveJournal } from "../../Utils/BackendRequester";
import { DirectoryListing } from "./DirectoryListing";

interface Props {}

export const Save: React.FC<Props> = () => {
  const [input, setInput] = useState("");
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
    if (input) {
      setLoading(true);
      saveJournal({ commitMsg: input }).then(updateDirtyFiles);
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
                  setInput(e.target.value);
                  setError("");
                }}
              />
              {error && (
                <Label color="red" pointing>
                  {error}
                </Label>
              )}
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
