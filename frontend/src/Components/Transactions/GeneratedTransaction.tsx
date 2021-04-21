import React from "react";
import { Breadcrumb, Card, Label, Popup } from "semantic-ui-react";
import {
  getAmount,
  getDate,
  getId,
  getMatchingAccount,
  getUnmatchingAccount,
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
  const accountSections = (negative
    ? getMatchingAccount(transaction, account)
    : getUnmatchingAccount(transaction, account)
  )
    ?.split(":")
    .map((v) => {
      return { key: v, content: v };
    });

  const amt = getAmount(transaction, account);
  const amount = negative ? -amt : amt;

  return (
    <Popup
      inverted
      size="mini"
      trigger={
        <Card fluid>
          <Card.Content>
            <Card.Header>{transaction.tdescription}</Card.Header>
            <Card.Meta>{getDate(transaction)}</Card.Meta>
            <Card.Description>
              <Breadcrumb icon="right angle" sections={accountSections} />
            </Card.Description>
            <Card.Description></Card.Description>
            <Label attached="bottom right" size="large" color={amount > 0 ? "green" : "red"}>
              {formatter.format(amount)}
            </Label>
          </Card.Content>
        </Card>
      }
    >
      <Popup.Content>{getId(transaction)} </Popup.Content>
    </Popup>
  );
};

export default GeneratedTransaction;
