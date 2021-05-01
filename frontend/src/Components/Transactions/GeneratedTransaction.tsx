import React from "react";
import { Breadcrumb, Header, Popup, Table } from "semantic-ui-react";
import { getDate, getId, getPostingAmount, HledgerTransaction, Posting } from "../../Models/HledgerTransaction";

interface Props {
  transaction: HledgerTransaction;
  account: string;
  // Multiply amount by -1 if true
  negative?: boolean;
}

const GeneratedTransaction: React.FC<Props> = ({ transaction, account, negative }) => {
  const accountSections = (account?: string) =>
    account?.split(":").map((v) => {
      return { key: v, content: v };
    });

  const posting = (posting: Posting, key: number) => {
    const amount = getPostingAmount(posting, negative);
    console.log(posting);

    return (
      <Popup
        key={key.toString()}
        disabled={!posting.pcomment}
        position="left center"
        trigger={
          <Table.Row>
            <Table.Cell>
              <Breadcrumb icon="right angle" sections={accountSections(posting.paccount)} />
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
        <Table singleLine>
          <Table.Header>
            <Table.HeaderCell colSpan={2}>{transaction.tdescription}</Table.HeaderCell>
          </Table.Header>
          {transaction.tpostings?.map(posting)}
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
