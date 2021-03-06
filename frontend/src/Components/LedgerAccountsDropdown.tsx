import React, { useContext, useMemo } from "react";
import { Form, StrictFormDropdownProps } from "semantic-ui-react";
import { AccountsContext } from "../Utils/AccountsContext";
import { toTitleCase } from "../Utils/TextUtils";

interface Props extends StrictFormDropdownProps {
  account: string;
  onEdit(newAccount: string): void;
}

const LedgerAccountsDropdown: React.FC<Props> = ({ account, onEdit, ...props }) => {
  const { accounts } = useContext(AccountsContext);
  return useMemo(
    () => (
      <Form.Dropdown
        {...props}
        placeholder="Account name..."
        fluid
        selection
        search
        allowAdditions
        additionLabel="New account: "
        value={account}
        text={toTitleCase(account)}
        options={accounts.map((account) => ({
          value: account,
          text: toTitleCase(account),
        }))}
        onChange={(_, data) => {
          onEdit(data.value?.toString() ?? "Error");
        }}
      />
    ),
    [account, accounts, onEdit, props]
  );
};
LedgerAccountsDropdown.displayName = "LedgerAccountsDropdown";

export default LedgerAccountsDropdown;
