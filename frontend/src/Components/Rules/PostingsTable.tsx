import React from "react";
import { Table } from "semantic-ui-react";
import { RulePosting } from "../../Models/Rule";
import PostingComponent from "./Posting";

interface Props {
  postings: RulePosting[];
}

const PostingsTable: React.FC<Props> = ({ postings }) => {
  return (
    <Table size="small" compact singleLine basic="very" celled>
      <Table.Header>
        <Table.Row>
          <Table.HeaderCell>Account</Table.HeaderCell>
          <Table.HeaderCell>Amount field</Table.HeaderCell>
          <Table.HeaderCell>Currency field</Table.HeaderCell>
          <Table.HeaderCell>Price amount field</Table.HeaderCell>
          <Table.HeaderCell>Price currency field</Table.HeaderCell>
          <Table.HeaderCell>Comment</Table.HeaderCell>
          <Table.HeaderCell>Negate</Table.HeaderCell>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {postings.map((p, index) => (
          <PostingComponent key={index} posting={p} />
        ))}
      </Table.Body>
    </Table>
  );
};

export default PostingsTable;
