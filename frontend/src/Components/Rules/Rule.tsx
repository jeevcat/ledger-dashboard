import React from "react";
import { Table, Button, Input, Dropdown, Label } from "semantic-ui-react";
import { RealTransactionField } from "../../Models/ImportRow";
import { Rule } from "../../Models/Rule";
import { toTitleCase } from "../../Utils/TextUtils";
import LedgerAccountsDropdown from "../LedgerAccountsDropdown";

interface Props {
  rule: Rule;
  error?: string;
  onEdit: (id: number, field: keyof Rule, value: string) => void;
  onDelete(rule: Rule): void;
  ruleFields: RealTransactionField[];
  accounts: string[];
}

const RuleComponent: React.FC<Props> = React.memo(
  ({ rule, error, onEdit, onDelete, ruleFields, accounts }) => {
    const ruleInput = (field: keyof Rule, error?: string) => {
      const input = (
        <Input
          fluid
          error={error !== undefined}
          value={rule[field]}
          onChange={(_, data) => {
            onEdit(rule.id, field, data.value);
          }}
        />
      );
      if (error) {
        return (
          <div>
            {input}
            <Label basic color="red" pointing>
              {error.toString()}
            </Label>
          </div>
        );
      } else {
        return input;
      }
    };
    const fieldDropdown = () => (
      <Dropdown
        placeholder="Select field..."
        selection
        search
        value={rule.matchFieldName}
        text={toTitleCase(rule.matchFieldName)}
        options={ruleFields.map((field) => ({
          value: field,
          text: toTitleCase(field.toString()),
        }))}
        onChange={(_, data) => {
          onEdit(rule.id, "matchFieldName", data.value?.toString() ?? "Error");
        }}
      />
    );

    return (
      <Table.Row>
        <Table.Cell verticalAlign="top">{ruleInput("priority")}</Table.Cell>
        <Table.Cell verticalAlign="top">{ruleInput("ruleName")}</Table.Cell>
        <Table.Cell verticalAlign="top">{fieldDropdown()}</Table.Cell>
        <Table.Cell verticalAlign="top">{ruleInput("matchFieldRegex", error)}</Table.Cell>
        <Table.Cell verticalAlign="top">
          <LedgerAccountsDropdown
            account={rule.targetAccount}
            onEdit={(newAccount: string) => onEdit(rule.id, "targetAccount", newAccount)}
            accounts={accounts}
          />
        </Table.Cell>
        <Table.Cell verticalAlign="top">{ruleInput("descriptionTemplate")}</Table.Cell>
        <Table.Cell verticalAlign="top" textAlign="center">
          <Button icon="delete" negative onClick={(_) => onDelete(rule)} />
        </Table.Cell>
      </Table.Row>
    );
  },
  (prevProps, nextProps) => {
    if (
      JSON.stringify(prevProps.rule) === JSON.stringify(nextProps.rule) &&
      prevProps.ruleFields.length === nextProps.ruleFields.length
    ) {
      return true; // props are equal
    }
    return false; // props are not equal -> update the component
  }
);
RuleComponent.displayName = "RuleComponent";

export default RuleComponent;
