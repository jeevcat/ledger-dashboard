import React, { useCallback, useContext, useEffect, useState } from "react";
import { Button, Container, Icon, Loader } from "semantic-ui-react";
import { Rule } from "../../Models/Rule";
import { AccountsContext } from "../../Utils/AccountsContext";
import { deleteRule, getRules, setRule } from "../../Utils/BackendRequester";
import RulesTable from "./RulesTable";

interface Props {}

type RuleErrors = { [rule: string]: string | undefined };

const Rules: React.FC<Props> = () => {
  const [isLoadingRules, setIsLoadingRules] = useState(false);
  const [rules, setRules] = useState<Rule[]>([]);
  const [errors, setErrors] = useState<RuleErrors>({});

  const { importAccount } = useContext(AccountsContext);

  const handleRuleSave = async (rule: Rule): Promise<boolean> => {
    return setRule(importAccount, rule).then((error) => {
      if (error) {
        setErrors((prevErrors) => ({ ...prevErrors, [rule._id?.$oid ?? "ERROR"]: error }));
        return true;
      } else {
        fetchRules();
        return false;
      }
    });
  };

  const handleRuleDelete = async (rule: Rule) => {
    deleteRule(rule).then(() => {
      fetchRules();
    });
  };

  const handleRuleNew = () => {
    const rule: Rule = {
      priority: 100,
      importerId: importAccount.id,
      ruleName: "NEW RULE",
      matchFieldName: "description",
      descriptionTemplate: "{{description}}",
      matchFieldRegex: "(?i)",
      postings: [
        {
          account: "?",
          negate: true,
        },
      ],
    };
    setRule(importAccount, rule).then(fetchRules);
  };

  const fetchRules = useCallback(() => {
    setIsLoadingRules(true);
    getRules(importAccount)
      .then((data: Rule[]) => {
        setRules(data);
        setIsLoadingRules(false);
      })
      .catch((e) => console.error(`Couldn't fetch rules ${e}`));
  }, [importAccount]);

  useEffect(() => {
    fetchRules();
  }, [fetchRules]);

  if (isLoadingRules) {
    return <Loader active />;
  } else {
    return (
      <>
        <RulesTable
          errors={errors}
          rules={rules}
          onSetRuleRequested={handleRuleSave}
          onDeleteRuleRequested={handleRuleDelete}
        />
        <br />
        <Container fluid textAlign="right">
          <Button primary icon labelPosition="right" onClick={handleRuleNew}>
            New
            <Icon name="add" />
          </Button>
        </Container>
      </>
    );
  }
};

export default Rules;
