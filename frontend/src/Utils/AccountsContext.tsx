import React, { createContext, useEffect, useState } from "react";
import { getAccounts } from "./BackendRequester";

export interface AccountsContextData {
  accounts: string[];
}

export const AccountsContext = createContext<AccountsContextData>({ accounts: [] });

export const AccountsContextComponent: React.FC = ({ children }) => {
  const [accounts, setAccounts] = useState<string[]>([]);

  const fetchAccounts = () => {
    getAccounts().then((data: string[]) => {
      setAccounts(data);
    });
  };

  useEffect(() => {
    fetchAccounts();
  }, []);

  return <AccountsContext.Provider value={{ accounts }}>{children}</AccountsContext.Provider>;
};
