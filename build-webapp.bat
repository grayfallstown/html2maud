rm index* trunk*
cd yew
echo "github does not allow usage of github-pages folder. Must be docs or root"
trunk build --release -d ../docs
cd ..
git status
