import React from "react";
import { Icon, Table } from "semantic-ui-react";
import { RulePosting } from "../../Models/Rule";
import AccountName from "../Transactions/AccountName";

interface Props {
  posting: RulePosting;
}

const PostingComponent: React.FC<Props> = React.memo(({ posting }) => {
  //const none = <Label basic>None</Label>;
  const none = <span style={{ color: "LightGray" }}>·</span>;
  return (
    <Table.Row>
      <Table.Cell>
        <AccountName account={posting.account} />
      </Table.Cell>
      <Table.Cell>
        <code>{posting.amountFieldName}</code>
      </Table.Cell>
      <Table.Cell textAlign="center">
        {posting.currencyFieldName ? <code>{posting.currencyFieldName}</code> : none}
      </Table.Cell>
      <Table.Cell textAlign="center">
        {" "}
        {posting.price ? <code>{posting.price?.amountFieldName}</code> : none}
      </Table.Cell>
      <Table.Cell textAlign="center">
        {" "}
        {posting.price ? <code>{posting.price?.currencyFieldName}</code> : none}
      </Table.Cell>
      <Table.Cell textAlign="center">{posting.comment ? posting.comment : none}</Table.Cell>
      <Table.Cell textAlign="center">
        <Icon name={posting.negate ? "check" : "x"} />
      </Table.Cell>
    </Table.Row>
  );
});
PostingComponent.displayName = "PostingComponent";

export default PostingComponent;
