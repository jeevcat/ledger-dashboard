import React, { useState } from "react";
import { Grid, Form, Segment, Button, Label } from "semantic-ui-react";
import { ping } from "../../Utils/BackendRequester";

interface Props {
  setApiKey(token: string): void;
}

export const Login: React.FC<Props> = ({ setApiKey }) => {
  const [input, setInput] = useState("");
  const [error, setError] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const success = await ping(input);
    if (success) {
      setApiKey(input);
    } else {
      setError(true);
    }
  };

  return (
    <Grid textAlign="center" style={{ height: "100vh" }} verticalAlign="middle">
      <Grid.Column style={{ maxWidth: 450 }}>
        <Form size="large" onSubmit={handleSubmit}>
          <Segment stacked>
            <Form.Field fluid icon="lock" iconPosition="left" error={error}>
              <input
                placeholder="API Key"
                type="password"
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

            <Button color="blue" fluid size="large">
              Login
            </Button>
          </Segment>
        </Form>
      </Grid.Column>
    </Grid>
  );
};
