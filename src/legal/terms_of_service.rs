use crate::prelude::*;

struct Tos;
impl Component for Tos {
    fn render(&self) -> String {
        r#"
        <div class="prose bg-slate-300 rounded p-2 md:p-4">
            <h1>Terms of Service</h1>
            <p>
              This document outlines the terms and conditions under which
              users may access and use the calorie counter services provided
              by Bean Count, a business based in New Jersey. By accessing or
              using the services, you agree to be bound by these terms. If you
              do not agree with any part of these terms, please do not use the
              website.
            </p>

            <h2>1. Acceptance of Terms</h2>
            <p>
              By using the calorie counter services provided by Bean Count,
              you acknowledge that you have read, understood, and agree to be
              bound by these terms of service.
            </p>

            <h2>2. Service Description</h2>
            <p>
              Bean Count provides a calorie counter tool that utilizes ChatGPT
              to estimate calorie counts for various foods. Users pay a
              monthly subscription fee of $1 to access and use this service.
            </p>

            <h2>3. User Accounts</h2>
            <p>
              Users must create an account to access the calorie counter
              services. You are responsible for maintaining the
              confidentiality of your account information, including your
              password.
            </p>

            <h2>4. Payment and Subscription</h2>
            <p>
              Users are required to pay a monthly subscription fee of $1 to
              access the calorie counter services. Payments are processed
              securely through Stripe. Subscription fees are non-refundable.
            </p>

            <h2>Subscription Cancellation</h2>
            <p>
              Users can manage their own subscription on Stripe. Users can
              cancel their subscription at any time. When the subscription
              period ends, access to the site will be revoked until continuation
              of payment. Users who have cancelled their subscription can resume
              their subscription at any time.
            </p>

            <h2>5. Accuracy of Calorie Estimates</h2>
            <p>
              While Bean Count strives to provide accurate calorie estimates,
              users acknowledge that the estimates provided are for
              informational purposes only and may not be entirely precise.
              It is advised to consult with a healthcare professional for
              personalized dietary advice.
            </p>

            <h2>6. User Conduct</h2>
            <p>
              Users agree to use the calorie counter services in a lawful and
              responsible manner. Any misuse or unauthorized use may result in
              the termination of your account.
            </p>

            <h2>7. Intellectual Property</h2>
            <p>
              The content, design, and intellectual property of Bean Count are
              protected by copyright and other intellectual property laws.
              Users may not reproduce, distribute, or create derivative works
              without the express written consent of Bean Count.
            </p>

            <h2>8. Limitation of Liability</h2>
            <p>
              Bean Count is not liable for any direct, indirect, incidental,
              consequential, or punitive damages arising out of the use or
              inability to use the calorie counter services. Note that LLM
              calorie estimates are not guaranteed to be accurate, and users
              should impose their own judgement on the estimates that this
              website generates.
            </p>

            <h2>9. Termination of Services</h2>
            <p>
              Bean Count reserves the right to terminate or suspend
              access to the calorie counter services at its discretion,
              without notice, for any reason, including but not limited to a
              breach of these terms.
            </p>

            <h2>10. Changes to Terms</h2>
            <p>
              Bean Count reserves the right to modify or update these terms of
              service at any time. Users will be notified of significant
              changes, and continued use of the services after such modifications
              constitutes acceptance of the updated terms.
            </p>

            <p>
              By using the calorie counter services provided by
              Bean Count, you agree to abide by these terms of
              service. If you have any questions or concerns, please contact us at
              <a href="mailto:jdevries3133@gmail.com">jdevries3133@gmail.com</a>.
            </p>
        </div>
        "#.into()
    }
}

pub async fn get_tos() -> impl IntoResponse {
    Page {
        title: "Terms of Service",
        children: &PageContainer { children: &Tos {} },
    }
    .render()
}
