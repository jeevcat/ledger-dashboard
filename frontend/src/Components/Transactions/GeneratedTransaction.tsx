import React from "react";
import { Header, Popup, Table } from "semantic-ui-react";
import { getDate, getId, getPostingAmount, HledgerTransaction, Posting } from "../../Models/HledgerTransaction";
import AccountName from "./AccountName";

interface Props {
  transaction: HledgerTransaction;
  account: string;
  // Multiply amount by -1 if true
  negative?: boolean;
}

const GeneratedTransaction: React.FC<Props> = ({ transaction, account, negative }) => {
  const posting = (posting: Posting, key: number) => {
    const amount = getPostingAmount(posting, negative);

    return (
      <Popup
        key={key.toString()}
        disabled={!posting.pcomment}
        position="left center"
        trigger={
          <Table.Row>
            <Table.Cell>
              <AccountName account={posting.paccount} />
            </Table.Cell>
            <Table.Cell textAlign="right" positive={amount.positive} negative={!amount.positive}>
              {amount.formatted}
            </Table.Cell>
          </Table.Row>
        }
      >
        {posting.pcomment}
      </Popup>
    );
  };

  return (
    <Popup
      position="top right"
      inverted
      size="mini"
      trigger={
        <Table singleLine compact="very">
          <Table.Header>
            <Table.Row>
              <Table.HeaderCell style={{ padding: ".4em .6em" }} colSpan={2}>
                {transaction.tdescription}
              </Table.HeaderCell>
            </Table.Row>
          </Table.Header>
          <Table.Body>{transaction.tpostings?.map(posting)}</Table.Body>
        </Table>
      }
    >
      <Popup.Content>
        <Header>{getDate(transaction)}</Header>
        {getId(transaction)}{" "}
      </Popup.Content>
    </Popup>
  );
};

export default GeneratedTransaction;
