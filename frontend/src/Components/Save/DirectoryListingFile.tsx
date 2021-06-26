import React from "react";
import { List } from "semantic-ui-react";

interface Props {
  path: string;
}

const leaf = (path: string): string | undefined => path.split("\\")?.pop()?.split("/").pop();

const trimTrailingSlashes = (path: string): string => path.replace(/[/\\]+$/, "");

export const DirectoryListingFile: React.FC<Props> = ({ path }) => {
  const filename = leaf(path);
  const folders = filename ? trimTrailingSlashes(path.replace(filename, "")) : undefined;
  return (
    <List.Item>
      <List.Icon name="file" size="large" verticalAlign="middle" />
      <List.Content>
        <List.Header>{filename}</List.Header>
        {folders && <List.Description>{folders}</List.Description>}
      </List.Content>
    </List.Item>
  );
};
