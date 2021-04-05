import React, { useContext } from "react";
import { Breadcrumb, Card, Label } from "semantic-ui-react";
import { getAmount, getDate, getTargetAccount, RecordedTransaction } from "../../Models/RecordedTransaction";
import { AccountsContext } from "../../Utils/AccountsContext";

const formatter = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "EUR",
});

interface Props {
  transaction: RecordedTransaction;
}

const GeneratedTransaction: React.FC<Props> = ({ transaction }) => {
  const {
    importAccount: { id: path },
  } = useContext(AccountsContext);

  const accountSections = getTargetAccount(transaction, path)
    ?.split(":")
    .map((v) => {
      return { key: v, content: v };
    });

  const amount = getAmount(transaction, path);

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
