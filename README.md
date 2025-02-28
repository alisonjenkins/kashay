# eks-auth

eks-auth is a replacement to the the `aws eks get-token` command. It's purpose is to speed up the
process of obtaining credentials when setup as part of your kube-config.

The default `aws eks get-token` is written in Python and on my machine takes typically about 525-562ms
to get credentials.

eks-auth on the other hand typically takes between 120-163ms because it is written in Rust and does not have
to load a Python interpreter and then interpret code to be able to get the credentials.

## Benchmark

Here is a benchmark of eks-creds running on my M3 Macbook Pro:

```bash
hyperfine --warmup 10 './target/release/eks-creds --cluster-name mycluster --aws-region us-east-1 myawsprofile'
Benchmark 1: ./target/release/eks-creds --cluster-name mycluster --aws-region us-east-1 myawsprofile
  Time (mean ± σ):     133.8 ms ±   8.2 ms    [User: 78.9 ms, System: 14.1 ms]
  Range (min … max):   128.0 ms … 163.4 ms    22 runs
```

and here is the equivalent AWS CLI command that is used by default in the kube-config:

```bash
hyperfine --warmup 10 'aws eks get-token --cluster-name mycluster --region us-east-1 --profile myawsprofile'
Benchmark 1: aws eks get-token --cluster-name mycluster --region us-east-1 --profile myawsprofile
  Time (mean ± σ):     538.5 ms ±  11.7 ms    [User: 255.8 ms, System: 145.0 ms]
  Range (min … max):   525.7 ms … 562.8 ms    10 runs
```

## License

This software is licensed under the [MIT license](LICENSE).
