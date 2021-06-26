import React from "react";
import { List } from "semantic-ui-react";
import { DirectoryListingFile } from "./DirectoryListingFile";

interface Props {
  paths: string[];
}

export const DirectoryListing: React.FC<Props> = ({ paths }) => {
  return (
    <List divided relaxed>
      {paths.map((path) => (
        <DirectoryListingFile key={path} path={path} />
      ))}
    </List>
  );
};
