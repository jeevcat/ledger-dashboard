import React from "react";
import { Breadcrumb } from "semantic-ui-react";

interface Props {
  account: string;
}

const accountSections = (account?: string) =>
  account?.split(":").map((v) => {
    return { key: v, content: v };
  });

const AccountName: React.FC<Props> = ({ account }) => {
  return <Breadcrumb icon="right angle" sections={accountSections(account)} />;
};

export default AccountName;
