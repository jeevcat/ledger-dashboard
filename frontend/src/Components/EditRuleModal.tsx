import React, { useContext, useState } from "react";
import { Button, Form, Header, Icon, Label, Modal, Segment } from "semantic-ui-react";
import { RealTransactionField } from "../Models/ImportRow";
import { Price, Rule, RulePosting } from "../Models/Rule";
import { AccountsContext } from "../Utils/AccountsContext";
import { toTitleCase } from "../Utils/TextUtils";
import LedgerAccountsDropdown from "./LedgerAccountsDropdown";

const NUMBER_FIELDS: (keyof Rule)[] = ["priority"];

interface Props {
  initialRule: Rule;
  error?: string;
  onSave: (rule: Rule) => Promise<boolean>;
}

const EditRuleModal: React.FC<Props> = ({ initialRule, error, onSave }) => {
  const {
    importAccount: { defaultColumns: ruleFields, amountColumns },
  } = useContext(AccountsContext);
  const [isOpen, setIsOpen] = useState<boolean>(false);
  const [isSaving, setIsSaving] = useState<boolean>(false);
  const [rule, setRule] = useState(initialRule);

  const updatePosting = (field: keyof RulePosting, postingIndex: number, value: any) =>
    setRule((prev) => ({
      ...prev,
      postings: [
        ...prev.postings.slice(0, postingIndex),
        { ...prev.postings[postingIndex], [field]: value },
        ...prev.postings.slice(postingIndex + 1),
      ],
    }));

  const updatePrice = (field: keyof RulePosting, postingIndex: number, value: any) => {
    if (!value) {
      updatePosting("price", postingIndex, undefined);
      return;
    }

    setRule((prev) => ({
      ...prev,
      postings: [
        ...prev.postings.slice(0, postingIndex),
        {
          ...prev.postings[postingIndex],
          price: {
            ...prev.postings[postingIndex].price,
            [field]: value,
          },
        },
        ...prev.postings.slice(postingIndex + 1),
      ],
    }));
  };

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

  const postingInput = (field: keyof RulePosting, postingIndex: number) => (
    <Form.Input
      fluid
      label={toTitleCase(field.toString())}
      value={rule.postings[postingIndex][field]}
      placeholder="None"
      onChange={(_, data) => updatePosting(field, postingIndex, data.value)}
    />
  );

  const postingCheckbox = (field: keyof RulePosting, postingIndex: number) => (
    <Form.Checkbox
      label={toTitleCase(field.toString())}
      checked={!!rule.postings[postingIndex][field]}
      onChange={(_, data) => updatePosting(field, postingIndex, data.checked)}
    />
  );

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

  const postingFieldDropdown = (field: keyof RulePosting, postingIndex: number, options: RealTransactionField[]) => (
    <Form.Dropdown
      label={toTitleCase(field.toString())}
      value={rule.postings[postingIndex][field] as string | undefined}
      placeholder="None"
      selection
      search
      clearable
      options={options.map((field) => ({
        value: field,
        text: toTitleCase(field.toString()),
      }))}
      onChange={(_, data) => updatePosting(field, postingIndex, data.value)}
    />
  );

  const priceFieldDropdown = (field: keyof Price, postingIndex: number) => {
    const price = rule.postings[postingIndex].price;
    return (
      <Form.Dropdown
        label={"Price " + toTitleCase(field.toString())}
        value={price ? price[field] : ""}
        placeholder="None"
        selection
        search
        clearable
        options={ruleFields.map((field) => ({
          value: field,
          text: toTitleCase(field.toString()),
        }))}
        onChange={(_, data) => updatePrice(field, postingIndex, data.value)}
      />
    );
  };

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
      size="large"
    >
      <Modal.Header>Edit rule</Modal.Header>
      <Modal.Content>
        <Form>
          <Form.Group widths="equal">
            {ruleInput("priority")}
            {ruleInput("ruleName")}
          </Form.Group>
          <Form.Group widths="equal">
            {fieldDropdown()}
            {ruleInput("matchFieldRegex", error)}
          </Form.Group>
          {ruleInput("descriptionTemplate")}
          <Header>Postings ({rule.postings.length})</Header>
          {rule.postings.map((posting, index) => (
            <Segment key={index}>
              <Label ribbon>{String.fromCharCode(65 + index)}</Label>
              <Form.Group widths="equal">
                <LedgerAccountsDropdown
                  label="Account"
                  account={posting.account}
                  onEdit={(newAccount: string) => updatePosting("account", index, newAccount)}
                />
                {postingInput("comment", index)}
              </Form.Group>
              <Form.Group widths="equal">
                {amountColumns.length > 1 && postingFieldDropdown("amountFieldName", index, amountColumns)}
                {postingFieldDropdown("currencyFieldName", index, ruleFields)}
              </Form.Group>
              <Form.Group widths="equal">
                {priceFieldDropdown("amountFieldName", index)}
                {priceFieldDropdown("currencyFieldName", index)}
              </Form.Group>
              {postingCheckbox("negate", index)}
              <Button
                negative
                basic
                icon
                labelPosition="right"
                onClick={() =>
                  setRule((prev) => ({
                    ...prev,
                    postings: [...prev.postings.slice(0, index), ...prev.postings.slice(index + 1)],
                  }))
                }
              >
                <Icon name="delete" />
                Remove posting
              </Button>
            </Segment>
          ))}
          <Button
            positive
            icon
            basic
            labelPosition="right"
            onClick={() =>
              setRule((prev) => ({
                ...prev,
                postings: [...prev.postings, { account: "?", negate: true }],
              }))
            }
          >
            <Icon name="add" />
            Add posting
          </Button>
        </Form>
      </Modal.Content>
      <Modal.Actions>
        <Button
          negative
          icon
          labelPosition="right"
          onClick={() => {
            setRule(initialRule);
            setIsOpen(false);
          }}
        >
          <Icon name="remove" />
          Cancel
        </Button>
        <Button
          positive
          icon
          labelPosition="right"
          disabled={JSON.stringify(rule) === JSON.stringify(initialRule)}
          loading={isSaving}
          onClick={() => {
            setIsSaving(true);
            onSave(rule)
              .then((success: boolean) => {
                if (success) {
                  setIsOpen(false);
                }
              })
              .finally(() => setIsSaving(false));
          }}
        >
          <Icon name="checkmark" />
          Record
        </Button>
      </Modal.Actions>
    </Modal>
  );
};

export default EditRuleModal;
