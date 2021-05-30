import React, { useCallback, useContext, useState } from "react";
import { Button, Icon, Modal } from "semantic-ui-react";
import { RealTransaction } from "../Models/ImportRow";
import { AccountsContext } from "../Utils/AccountsContext";
import { generateSingleTransaction } from "../Utils/BackendRequester";
import RecordTransactionModalContent from "./RecordTransactionModalContent";

interface Props {
  realTransaction: RealTransaction;
  onWrite: () => void;
}

const RecordTransactionModal: React.FC<Props> = ({ realTransaction, onWrite }) => {
  const { importAccount } = useContext(AccountsContext);

  const [isOpen, setIsOpen] = useState<boolean>(false);
  const [account, setAccount] = useState("");
  const [descriptionTemplate, setDescriptionTemplate] = useState("{{description}}");

  const generateTransaction = useCallback(
    (shouldWrite: boolean) =>
      generateSingleTransaction(importAccount, {
        descriptionTemplate,
        sourceTransaction: realTransaction,
        postings: [{ account }],
        shouldWrite,
      }),
    [account, descriptionTemplate, importAccount, realTransaction]
  );

  return (
    <Modal
      trigger={
        <Button
          primary
          icon="write"
          onClick={() => {
            setIsOpen(true);
          }}
        />
      }
      size="large"
      open={isOpen}
    >
      <Modal.Header>Record transaction</Modal.Header>
      <RecordTransactionModalContent
        realTransaction={realTransaction}
        account={account}
        onAccountChange={setAccount}
        descriptionTemplate={descriptionTemplate}
        onDescriptionTemplateChange={setDescriptionTemplate}
      />
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
