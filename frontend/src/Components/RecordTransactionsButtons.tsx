import React, { useState } from "react";
import { Button, Icon } from "semantic-ui-react";
import { ImportAccount } from "../Models/ImportAccount";
import { writeGeneratedTransactions } from "../Utils/BackendRequester";

interface Props {
  account: ImportAccount;
  onGenerate(): void;
}
export const RecordTransactionsButton: React.FC<Props> = ({ account, onGenerate }) => {
  const [isLoading, setIsLoading] = useState(false);

  return (
    <Button
      primary
      basic
      icon
      labelPosition="right"
      disabled={isLoading}
      loading={isLoading}
      onClick={() => {
        setIsLoading(true);
        writeGeneratedTransactions(account).then(() => {
          setIsLoading(false);
          onGenerate();
        });
      }}
    >
      Record generated transactions
      <Icon name="file" />
    </Button>
  );
};
