Note that this is a project I'm hoping to make a few bucks off of, so I will not
accept PRs - both because it would be legally ambiguous, and also clearly
unethical to accept free work for a not-free service.

Note also that all rights to this source code are reserved. The code is open
source for the purpose of education and transparency, but I issue no license for
any other use. For example, it's OK to run this code on your machine to learn;
it's not OK to deploy a persistent clone of this service on a domain you own
where users on the internet could sign up.

Also, note that a lot of the supporting infrastructure behind this project is
free and open source on my GitHub;

- the [tech stack (PHAT Stack)](https://github.com/jdevries3133/phat_stack) I
  put together for this project
  - includes authentication, secure sessions, and CI/CD
- the [Terraform IaC
  module](https://github.com/jdevries3133/terraform-kubernetes-basic-deployment)
  used to deploy this application into my Kubernetes cluster
- all the software behind [my homelab cluster
  itself](https://github.com/jdevries3133/homelab_cluster)

Details about setting up the development environment for this project are in
./HACKING.md, should you wish to learn more!
