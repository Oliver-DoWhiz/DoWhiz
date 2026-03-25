# Account Creation

Use this file only after the route is known and the user has said they want account help.

## Global rules

- Ask the account question only after the route and the missing-document picture are clear, unless the user already volunteered account details.
- Ask whether the user already has an account before offering to create one.
- If the user already has an account, confirm the login email or username and whether they want to log in themselves or want guided browser help after they authenticate.
- Get explicit permission before opening a signup flow or entering user data.
- For agent-created signup, collect the exact email address and phone number the user wants tied to the account, even if the site may not require both on the first screen.
- Ask the user to handle OTP, MFA, or email verification themselves.
- Prefer user-set passwords over agent-generated passwords.
- If you must generate a password, generate a strong temporary password and tell the user to rotate it immediately.
- Return credentials only in the conversation response. Do not save them to files, notes, or memory.
- Do not ask for or reuse IRS Online Account credentials inside this workflow.

## Credential return format

After account creation, reply with:

- platform used
- login email or username
- password or temporary password
- any verification step the user still must complete
- whether you are continuing into filing or waiting on the user

## Sprintax account creation

### Minimum details to request

- first name
- last name
- email address
- phone number
- desired password, or permission for a temporary password

### Useful extras to request if the user wants a smoother filing handoff

- school, employer, or organization code if the user already has one
- tax year and route summary

### Workflow

1. Confirm the case is an NRA or NRA-leaning case that should use Sprintax.
2. Ask whether the user already has a Sprintax account.
3. If not, ask whether they want the agent to create one.
4. Collect the minimum fields.
5. Open the official Sprintax signup flow.
6. Create the account.
7. Return the username or email used and the password or temporary password.
8. Ask the user to verify the email and rotate the password if you generated it.
9. If the user also asked for filing help, continue into Sprintax return preparation after verification steps are complete.

### Known official signup entry points

- https://taxprep.sprintax.com/create-account.html
- https://www.sprintax.com/dashboard/signup/
- https://www.sprintax.com/returns/

Sprintax may block some script-style requests. If a page returns 403 outside a browser, open it in a browser before deciding it is unavailable.

## TurboTax or Intuit account creation

### Minimum details to request

- first name
- last name
- email address
- phone number
- desired password, or permission for a temporary password

### Workflow

1. Confirm the case should not use Sprintax.
2. Confirm whether this is TurboTax individual or business.
3. Ask whether the user already has an Intuit or TurboTax account.
4. If not, ask whether they want the agent to create one.
5. Collect the minimum fields.
6. Open the official Intuit or TurboTax signup flow.
7. Create the account.
8. Return the email or username used and the password or temporary password.
9. Ask the user to complete verification and rotate the password if needed.
10. If the user also asked for filing help, continue into TurboTax after verification steps are complete.

### Known official entry points

- https://accounts.intuit.com/
- https://accounts.intuit.com/signup.html
- https://turbotax.intuit.com/personal-taxes/online/
- https://turbotax.intuit.com/small-business-taxes/
- https://turbotax.intuit.com/small-business-taxes/cd-download/

## Browser automation checklist

Before pressing the final submit button on signup:

- confirm the exact email address
- confirm the exact phone number if shown
- confirm whether marketing opt-ins should stay unchecked
- confirm whether the user wants password recovery through that email or phone
- return the final login identifier and password to the user once the account exists

Before pressing the final submit button on filing:

- summarize the route and major facts one more time
- call out any unanswered tax question
- ask the user for explicit approval to continue
