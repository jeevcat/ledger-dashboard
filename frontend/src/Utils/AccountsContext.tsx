import React, { createContext, useEffect, useState } from "react";
import { ImportAccount, ImportAccounts } from "../Models/ImportAccount";
import { getAccounts } from "./BackendRequester";

export interface AccountsContextData {
  accounts: string[];
  importAccount: ImportAccount;
}

export const AccountsContext = createContext<AccountsContextData>({ accounts: [], importAccount: ImportAccounts[0] });

interface Props {
  importAccount: ImportAccount;
}

export const AccountsContextComponent: React.FC<Props> = ({ importAccount, children }) => {
  const [accounts, setAccounts] = useState<string[]>([]);

  const fetchAccounts = () => {
    getAccounts()
      .then((data: string[]) => {
        setAccounts(data);
      })
      .catch((e) => console.error(`Couldn't fetch accounts: ${e}`));
  };

  useEffect(() => {
    fetchAccounts();
  }, []);

  return <AccountsContext.Provider value={{ accounts, importAccount }}>{children}</AccountsContext.Provider>;
};
