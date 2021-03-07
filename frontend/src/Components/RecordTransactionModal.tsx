import React, { useState, useCallback } from "react";
import { Header, Modal, Button, Icon, Input, Loader, Grid } from "semantic-ui-react";
import { Transaction } from "../Models/Transaction";
import TransactionSummary from "./TransactionSummary";
import GeneratedTransaction from "./Transactions/GeneratedTransaction";
import { generateSingleTransaction } from "../Utils/BackendRequester";
import LedgerAccountsDropdown from "./LedgerAccountsDropdown";
import { RealTransaction } from "../Models/ImportRow";

interface Props {
  realTransaction: RealTransaction;
  accounts: string[];
  onWrite: () => void;
}

const RecordTransactionModal: React.FC<Props> = ({ realTransaction, accounts, onWrite }) => {
  const [isOpen, setIsOpen] = useState<boolean>(false);
  const [generatedTransaction, setGeneratedTransaction] = useState<Transaction | undefined>(undefined);
  const [account, setAccount] = useState("");
  const [descriptionTemplate, setDescriptionTemplate] = useState(realTransaction.referenceText);

  const generateTransaction = useCallback(
    (shouldWrite: boolean) =>
      generateSingleTransaction({ account, descriptionTemplate, sourceTransaction: realTransaction, shouldWrite }),
    [account, descriptionTemplate, realTransaction]
  );

  const updatePreview = useCallback(
    () =>
      generateSingleTransaction({
        account,
        descriptionTemplate,
        sourceTransaction: realTransaction,
        shouldWrite: false,
      }).then((response: Transaction) => {
        setGeneratedTransaction(response);
      }),
    [account, descriptionTemplate, realTransaction]
  );

  return (
    <Modal
      trigger={
        <Button
          primary
          icon="write"
          onClick={() => {
            setIsOpen(true);
            updatePreview();
          }}
        />
      }
      open={isOpen}
    >
      <Modal.Header>Record transaction</Modal.Header>
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
                onChange={(_, data) => {
                  setDescriptionTemplate(data.value);
                  updatePreview();
                }}
              />
              <br />
              <LedgerAccountsDropdown account={account} accounts={accounts} onEdit={setAccount} />
            </Grid.Column>
            <Grid.Column>
              {generatedTransaction ? <GeneratedTransaction transaction={generatedTransaction} /> : <Loader />}
            </Grid.Column>
          </Grid>
        </Modal.Description>
      </Modal.Content>
      <Modal.Actions>
        <Button
          color="red"
          onClick={() => {
            setIsOpen(false);
          }}
        >
          <Icon name="remove" /> Cancel
        </Button>
        <Button
          color="green"
          onClick={() => {
            setIsOpen(false);
            generateTransaction(true);
            onWrite();
          }}
        >
          <Icon name="checkmark" /> Record
        </Button>
      </Modal.Actions>
    </Modal>
  );
};

export default RecordTransactionModal;
