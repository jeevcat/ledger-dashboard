import React from "react";
import { Dropdown, StrictDropdownProps } from "semantic-ui-react";
import { toTitleCase } from "../Utils/TextUtils";

interface Props extends StrictDropdownProps {
  account: string;
  accounts: string[];
  onEdit(newAccount: string): void;
}

const LedgerAccountsDropdown: React.FC<Props> = React.memo(({ account, accounts, onEdit, ...props }) => (
  <Dropdown
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
));
LedgerAccountsDropdown.displayName = "LedgerAccountsDropdown";

export default LedgerAccountsDropdown;
