import React, { useEffect, useState } from "react";
import { Grid, Header, Input, Loader, Modal } from "semantic-ui-react";
import { RealTransaction } from "../Models/ImportRow";
import { Transaction } from "../Models/Transaction";
import { generateSingleTransaction } from "../Utils/BackendRequester";
import LedgerAccountsDropdown from "./LedgerAccountsDropdown";
import GeneratedTransaction from "./Transactions/GeneratedTransaction";
import TransactionSummary from "./TransactionSummary";

interface Props {
  realTransaction: RealTransaction;
  sourceAccount: string;
  accounts: string[];
  account: string;
  onAccountChange(account: string): void;
  descriptionTemplate: string;
  onDescriptionTemplateChange(descriptionTemplate: string): void;
}

const RecordTransactionModalContent: React.FC<Props> = ({
  realTransaction,
  accounts,
  sourceAccount,
  account,
  onAccountChange,
  descriptionTemplate,
  onDescriptionTemplateChange,
}) => {
  const [generatedTransaction, setGeneratedTransaction] = useState<Transaction | undefined>(undefined);

  useEffect(() => {
    setGeneratedTransaction(undefined);
    generateSingleTransaction({
      account,
      descriptionTemplate,
      sourceTransaction: realTransaction,
      shouldWrite: false,
    })
      .catch((reason: any) => {
        console.warn("Couldn't generate: " + reason);
      })
      .then((response: Transaction) => {
        setGeneratedTransaction(response);
      });
  }, [account, descriptionTemplate, realTransaction]);

  return (
    <Modal.Content>
      <Modal.Description>
        <Header>Transaction summary</Header>
        <TransactionSummary realTransaction={realTransaction} columns="3" />
        <Header>Generated transaction</Header>
        <Grid columns="2" divided verticalAlign="middle">
          <Grid.Column>
            <Input
              fluid
              value={descriptionTemplate}
              label="Description"
              onChange={(_: any, data: any) => {
                onDescriptionTemplateChange(data.value);
              }}
            />
            <br />
            <LedgerAccountsDropdown
              account={account}
              accounts={accounts}
              onEdit={(newAccount: string) => {
                onAccountChange(newAccount);
              }}
            />
          </Grid.Column>
          <Grid.Column>
            {generatedTransaction ? (
              <GeneratedTransaction transaction={generatedTransaction} sourceAccount={sourceAccount} />
            ) : (
              <Loader />
            )}
          </Grid.Column>
        </Grid>
      </Modal.Description>
    </Modal.Content>
  );
};

export default RecordTransactionModalContent;
