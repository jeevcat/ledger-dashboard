import React, { useState } from "react";
import { Button, Form, Grid, Header, Label, Segment } from "semantic-ui-react";
import { DirectoryListing } from "./DirectoryListing";

interface Props {}

export const Save: React.FC<Props> = () => {
  const [input, setInput] = useState("");
  const [error, setError] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
  };
  return (
    <Grid textAlign="center" verticalAlign="middle" style={{ height: "100vh", margin: 0 }}>
      <Grid.Column style={{ maxWidth: 800 }} textAlign="left">
        <Header as="h1">Commit changes</Header>
        <Form size="large" onSubmit={handleSubmit}>
          <Segment stacked>
            <Form.Field fluid error={error}>
              <input
                placeholder="Commit message"
                type="text"
                onChange={(e) => {
                  setInput(e.target.value);
                  setError(false);
                }}
              />
              {error && (
                <Label color="red" pointing>
                  Login failed
                </Label>
              )}
            </Form.Field>
            <DirectoryListing paths={[String.raw`D:\Projects\ledger-dashboard\frontend\src\Components\Save\Save.tsx`, "/usr/bin/local/my_exe.bin"]} />

            <Button color="blue" fluid size="large">
              Commit changes
            </Button>
          </Segment>
        </Form>
      </Grid.Column>
    </Grid>
  );
};
