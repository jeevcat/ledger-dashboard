import React, { useContext, useState } from "react";
import { Button, Form, Icon, Label, Modal } from "semantic-ui-react";
import { Rule } from "../Models/Rule";
import { AccountsContext } from "../Utils/AccountsContext";
import { toTitleCase } from "../Utils/TextUtils";
import LedgerAccountsDropdown from "./LedgerAccountsDropdown";

const NUMBER_FIELDS: (keyof Rule)[] = ["priority"];

interface Props {
  initialRule: Rule;
  error?: string;
  onSave: (rule: Rule) => void;
}

const EditRuleModal: React.FC<Props> = ({ initialRule, error, onSave }) => {
  const {
    importAccount: { defaultColumns: ruleFields },
  } = useContext(AccountsContext);
  const [isOpen, setIsOpen] = useState<boolean>(false);
  const [rule, setRule] = useState(initialRule);

  const ruleInput = (field: keyof Rule, error?: string) => {
    const input = (
      <Form.Input
        fluid
        label={toTitleCase(field.toString())}
        error={error !== undefined}
        value={rule[field]}
        onChange={(_, data) => {
          let newValue: number | string = data.value;
          if (NUMBER_FIELDS.includes(field)) {
            newValue = parseInt(data.value) ?? data.value;
          }
          setRule((prev) => ({ ...prev, [field]: newValue }));
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
    <Form.Dropdown
      label="Match on field"
      placeholder="Select field..."
      selection
      search
      value={rule.matchFieldName}
      text={toTitleCase(rule.matchFieldName)}
      options={ruleFields.map((field) => ({
        value: field,
        text: toTitleCase(field.toString()),
      }))}
      onChange={(_, data) => setRule((prev) => ({ ...prev, matchFieldName: data.value?.toString() ?? "Error" }))}
    />
  );

  return (
    <Modal
      trigger={
        <Button
          size="mini"
          compact
          primary
          icon="edit"
          onClick={() => {
            setIsOpen(true);
          }}
        />
      }
      open={isOpen}
    >
      <Modal.Header>Edit rule</Modal.Header>
      <Modal.Content>
        <Form>
          <Form.Group widths="equal">
            {ruleInput("priority")}
            {ruleInput("ruleName")}
          </Form.Group>
          {fieldDropdown()}
          {ruleInput("matchFieldRegex", error)}
          <LedgerAccountsDropdown
            label="Target account"
            account={rule.postings[0].account}
            onEdit={(newAccount: string) =>
              setRule((prev) => ({
                ...prev,
                postings: [{ ...prev.postings[0], account: newAccount }, ...prev.postings.slice(1)],
              }))
            }
          />
          {ruleInput("descriptionTemplate")}
        </Form>
      </Modal.Content>
      <Modal.Actions>
        <Button
          color="red"
          icon
          labelPosition="right"
          onClick={() => {
            setIsOpen(false);
          }}
        >
          <Icon name="remove" /> Cancel
        </Button>
        <Button
          color="green"
          icon
          labelPosition="right"
          disabled={JSON.stringify(rule) === JSON.stringify(initialRule)}
          onClick={() => {
            setIsOpen(false);
            onSave(rule);
          }}
        >
          <Icon name="checkmark" /> Record
        </Button>
      </Modal.Actions>
    </Modal>
  );
};

export default EditRuleModal;
