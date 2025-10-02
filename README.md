`cargo run <directory to watch>`

Then try to run 

```
echo hey > hey.txt
sleep 2
rm hey.txt
```

I'm seeing events emitted in a confusing order.
