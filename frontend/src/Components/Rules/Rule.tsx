import React from "react";
import { Table, Button, Label } from "semantic-ui-react";
import { Rule } from "../../Models/Rule";
import EditRuleModal from "../EditRuleModal";
import AccountName from "../Transactions/AccountName";

interface Props {
  rule: Rule;
  error?: string;
  onSet: (rule: Rule) => void;
  onDelete(rule: Rule): void;
}

const RuleComponent: React.FC<Props> = React.memo(({ rule, error, onSet, onDelete }) => {
  return (
    <Table.Row>
      <Table.Cell>{rule.priority}</Table.Cell>
      <Table.Cell>
        <Label color="blue">{rule.ruleName}</Label>
      </Table.Cell>
      <Table.Cell>{rule.descriptionTemplate}</Table.Cell>
      <Table.Cell>{rule.matchFieldName}</Table.Cell>
      <Table.Cell>{rule.matchFieldRegex}</Table.Cell>
      <Table.Cell>
        <AccountName account={rule.postings[0].account} />
      </Table.Cell>
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
