import React from "react";
import { List, SemanticICONS } from "semantic-ui-react";

const icons: { [key: string]: SemanticICONS } = {
  ledger: "book",
  journal: "book",
  yml: "database",
};

const leaf = (path: string): string | undefined => path.split("\\")?.pop()?.split("/").pop();

const extension = (filename: string): string | undefined => filename.split(".")?.pop();

const getIcon = (filename?: string): SemanticICONS => {
  if (filename) {
    const ext = extension(filename);
    if (ext) {
      const icon = icons[ext];
      if (icon) {
        return icon;
      }
    }
  }
  return "file outline";
};

const trimTrailingSlashes = (path: string): string => path.replace(/[/\\]+$/, "");

interface Props {
  path: string;
}
export const DirectoryListingFile: React.FC<Props> = ({ path }) => {
  const filename = leaf(path);
  const folders = filename ? trimTrailingSlashes(path.replace(filename, "")) : undefined;
  return (
    <List.Item>
      <List.Icon name={getIcon(filename)} size="large" verticalAlign="middle" />
      <List.Content>
        <List.Header>{filename}</List.Header>
        {folders && <List.Description>{folders}</List.Description>}
      </List.Content>
    </List.Item>
  );
};
