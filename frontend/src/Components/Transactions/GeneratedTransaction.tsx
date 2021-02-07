import React from "react";
import { Transaction, getDate, getAmount, getTargetAccount } from "../../Models/Transaction";
import { Label, Card, Breadcrumb } from "semantic-ui-react";

const formatter = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "EUR",
});

interface Props {
  transaction: Transaction;
}

const GeneratedTransaction: React.FC<Props> = ({ transaction }) => {
  const accountSections = getTargetAccount(transaction, "N26")
    ?.split(":")
    .map((v) => {
      return { key: v, content: v };
    });

  const amount = getAmount(transaction, "N26");

  return (
    <Card fluid>
      <Card.Content>
        <Card.Header>{transaction.tdescription}</Card.Header>
        <Card.Meta>{getDate(transaction)}</Card.Meta>
        <Card.Description>
          <Breadcrumb icon="right angle" sections={accountSections} />
        </Card.Description>
        <Label attached="bottom right" size="large" color={amount > 0 ? "green" : "red"}>
          {formatter.format(amount)}
        </Label>
      </Card.Content>
    </Card>
  );
};

export default GeneratedTransaction;
