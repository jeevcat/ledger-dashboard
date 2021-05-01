import React from "react";
import { Breadcrumb, Card, Label, Popup, Table } from "semantic-ui-react";
import {
  getAmount,
  getDate,
  getId,
  getMatchingAccount,
  getUnmatchingAccounts,
  HledgerTransaction,
} from "../../Models/HledgerTransaction";

const formatter = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "EUR",
});

interface Props {
  transaction: HledgerTransaction;
  account: string;
  // Multiply amount by -1 if true
  negative?: boolean;
}

const GeneratedTransaction: React.FC<Props> = ({ transaction, account, negative }) => {
  const matching = getMatchingAccount(transaction, account);
  const unmatching = getUnmatchingAccounts(transaction, account);
  const accountSections = (account?: string) =>
    account?.split(":").map((v) => {
      return { key: v, content: v };
    });

  const posting = (account?: string) => {
    if (!account) return <div />;

    const amt = getAmount(transaction, account);
    const amount = negative ? -amt : amt;

    return (
      <Table.Row>
        <Table.Cell>
          <Breadcrumb icon="right angle" sections={accountSections(account)} />
        </Table.Cell>
        <Table.Cell textAlign="right" positive={amount > 0} negative={amount < 0}>
          {formatter.format(amount)}
        </Table.Cell>
      </Table.Row>
    );
  };

  return (
    <Popup
      inverted
      size="mini"
      trigger={
        <Card fluid>
          <Card.Content>
            <Card.Header>{transaction.tdescription}</Card.Header>
            <Card.Description>
              <Table>
                {posting(matching)}
                {unmatching.map(posting)}
              </Table>
            </Card.Description>
            <Label attached="top right">{getDate(transaction)}</Label>
          </Card.Content>
        </Card>
      }
    >
      <Popup.Content>{getId(transaction)} </Popup.Content>
    </Popup>
  );
};

export default GeneratedTransaction;
