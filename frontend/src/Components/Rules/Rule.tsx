import React from "react";
import { Button, Label, Table } from "semantic-ui-react";
import { Rule } from "../../Models/Rule";
import EditRuleModal from "../EditRuleModal";
import AccountName from "../Transactions/AccountName";
import PostingsTable from "./PostingsTable";

interface Props {
  rule: Rule;
  error?: string;
  onSet: (rule: Rule) => void;
  onDelete(rule: Rule): void;
}

const RuleComponent: React.FC<Props> = React.memo(({ rule, error, onSet, onDelete }) => {
  const postings =
    rule.postings.length < 2 ? (
      <AccountName account={rule.postings[0].account} />
    ) : (
      <PostingsTable postings={rule.postings} />
    );
  return (
    <Table.Row>
      <Table.Cell>{rule.priority}</Table.Cell>
      <Table.Cell>
        <Label color="blue">{rule.ruleName}</Label>
      </Table.Cell>
      <Table.Cell>
        <code>{rule.descriptionTemplate}</code>
      </Table.Cell>
      <Table.Cell>
        <code>{rule.matchFieldName}</code>
      </Table.Cell>
      <Table.Cell>
        <code>{rule.matchFieldRegex}</code>
      </Table.Cell>
      <Table.Cell>{postings}</Table.Cell>
      <Table.Cell textAlign="center">
        <Button compact size="mini" icon="delete" negative onClick={(_) => onDelete(rule)} />
      </Table.Cell>
      <Table.Cell textAlign="center">
        <EditRuleModal initialRule={rule} error={error} onSave={onSet} />
      </Table.Cell>
    </Table.Row>
  );
});
RuleComponent.displayName = "RuleComponent";

export default RuleComponent;
