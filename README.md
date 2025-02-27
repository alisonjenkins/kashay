# eks-auth

eks-auth is a replacement to the the `aws eks get-token` command. It's purpose is to speed up the
process of obtaining credentials when setup as part of your kube-config.

The default `aws eks get-token` is written in Python and on my machine takes typically about 1
second to get credentials.

eks-auth on the other hand typically takes about 200ms because it is written in Rust and does not have
to load a Python interpreter and then interpret code to be able to get the credentials.
