import React, { useContext } from "react";
import { Breadcrumb, Card, Label, Popup } from "semantic-ui-react";
import { getAmount, getDate, getId, getTargetAccount, HledgerTransaction } from "../../Models/HledgerTransaction";
import { AccountsContext } from "../../Utils/AccountsContext";

const formatter = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "EUR",
});

interface Props {
  transaction: HledgerTransaction;
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
