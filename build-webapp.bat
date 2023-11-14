rm index* trunk*
cd yew
echo "github does not allow usage of github-pages folder. Must be docs or root"
trunk build --release --public-url "/html2maud/" -d ../docs
cd ..
git status
