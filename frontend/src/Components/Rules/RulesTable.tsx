import React from "react";
import { Button, Container, Icon, Table } from "semantic-ui-react";
import { RealTransactionField } from "../../Models/ImportRow";
import { Rule } from "../../Models/Rule";
import RuleComponent from "./Rule";

type RuleErrors = { [rule: number]: string | undefined };

interface Props {
  onDeleteRuleRequested: (rule: Rule) => void;
  onEditRuleRequested: (id: number, field: keyof Rule, value: string) => void;
  onNewRuleRequested: () => void;
  rules: Rule[];
  errors: RuleErrors;
  ruleFields: RealTransactionField[];
}

const RulesTable: React.FC<Props> = ({
  onDeleteRuleRequested,
  onEditRuleRequested,
  onNewRuleRequested,
  rules,
  errors,
  ruleFields,
}) => {
  const ruleSort = (a: Rule, b: Rule) => {
    if (a.priority < b.priority) {
      return -1;
    }
    if (a.priority > b.priority) {
      return 1;
    }
    if (a.ruleName < b.ruleName) {
      return -1;
    }
    if (a.ruleName > b.ruleName) {
      return 1;
    }
    return 0;
  };

  return (
    <Container fluid>
      <Table>
        <Table.Header>
          <Table.Row>
            <Table.HeaderCell width="1">Priority</Table.HeaderCell>
            <Table.HeaderCell width="2">Name</Table.HeaderCell>
            <Table.HeaderCell width="2">Field name</Table.HeaderCell>
            <Table.HeaderCell width="2">Field regex</Table.HeaderCell>
            <Table.HeaderCell>Account</Table.HeaderCell>
            <Table.HeaderCell>Description template</Table.HeaderCell>
            <Table.HeaderCell width="1" />
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {rules.sort(ruleSort).map((r, index) => (
            <RuleComponent
              key={index}
              rule={r}
              error={errors[r.id]}
              onEdit={onEditRuleRequested}
              onDelete={onDeleteRuleRequested}
            />
          ))}
          <Table.Row>
            <Table.Cell colSpan="7">
              <Button primary icon labelPosition="right" onClick={onNewRuleRequested}>
                New
                <Icon name="add" />
              </Button>
            </Table.Cell>
          </Table.Row>
        </Table.Body>
      </Table>
    </Container>
  );
};

export default RulesTable;
