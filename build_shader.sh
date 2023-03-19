target=src/renderer/window_handler/winit_vulkan_handler/shader.rs
cat ./resources/shaders/shaders.pre > $target
cat ./resources/shaders/ray_tracer.comp | sed -n '1,/.*RECURSION.*/ p' >> $target
for (( i = 0; i < 17; i++ )); do
    cat resources/shaders/recursive.rec | sed -e "s/NN/$i/g" -e "s/MMM/$((i+1))/g" >> $target
done

for (( i = 0; i < 17; i++ )); do
    echo "}" >> $target
done
cat ./resources/shaders/ray_tracer.comp | sed '1,/.*RECURSION.*/d' >> $target
cat ./resources/shaders/shaders.post >> $target