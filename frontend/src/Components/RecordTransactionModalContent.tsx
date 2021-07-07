import React, { useContext, useEffect, useState } from "react";
import { Grid, Header, Input, Loader, Modal } from "semantic-ui-react";
import { HledgerTransaction } from "../Models/HledgerTransaction";
import { RealTransaction } from "../Models/ImportRow";
import { AccountsContext } from "../Utils/AccountsContext";
import { generateSingleTransaction } from "../Utils/BackendRequester";
import LedgerAccountsDropdown from "./LedgerAccountsDropdown";
import GeneratedTransaction from "./Transactions/GeneratedTransaction";
import TransactionSummary from "./TransactionSummary";

interface Props {
  realTransaction: RealTransaction;
  account: string;
  onAccountChange(account: string): void;
  descriptionTemplate: string;
  onDescriptionTemplateChange(descriptionTemplate: string): void;
}

const RecordTransactionModalContent: React.FC<Props> = ({
  realTransaction,
  account,
  onAccountChange,
  descriptionTemplate,
  onDescriptionTemplateChange,
}) => {
  const { importAccount } = useContext(AccountsContext);

  const [generatedTransaction, setGeneratedTransaction] = useState<HledgerTransaction | undefined>(undefined);

  useEffect(() => {
    setGeneratedTransaction(undefined);
    generateSingleTransaction(importAccount, {
      descriptionTemplate,
      sourceTransaction: realTransaction,
      postings: [{ account, negate: true }],
      shouldWrite: false,
    })
      .catch((reason: any) => {
        console.warn("Couldn't generate: " + reason);
      })
      .then((response: HledgerTransaction) => {
        setGeneratedTransaction(response);
      });
  }, [account, descriptionTemplate, importAccount, realTransaction]);

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
              onEdit={(newAccount: string) => {
                onAccountChange(newAccount);
              }}
            />
          </Grid.Column>
          <Grid.Column>
            {generatedTransaction ? (
              <GeneratedTransaction transaction={generatedTransaction} account={importAccount.id} />
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
